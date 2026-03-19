import { ComponentFixture, TestBed } from '@angular/core/testing';
import { HubFederationsComponent } from './hub-federations.component';

describe('HubFederationsComponent', () => {
  let component: HubFederationsComponent;
  let fixture: ComponentFixture<HubFederationsComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [HubFederationsComponent]
    }).compileComponents();

    fixture = TestBed.createComponent(HubFederationsComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
