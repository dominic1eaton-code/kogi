import { ComponentFixture, TestBed } from '@angular/core/testing';
import { HubDistributionComponent } from './hub-distribution.component';

describe('HubDistributionComponent', () => {
  let component: HubDistributionComponent;
  let fixture: ComponentFixture<HubDistributionComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [HubDistributionComponent]
    }).compileComponents();

    fixture = TestBed.createComponent(HubDistributionComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
