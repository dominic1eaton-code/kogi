import { ComponentFixture, TestBed } from '@angular/core/testing';
import { HubResourceCrowdfundComponent } from './hub-resource-crowdfund.component';

describe('HubResourceCrowdfundComponent', () => {
  let component: HubResourceCrowdfundComponent;
  let fixture: ComponentFixture<HubResourceCrowdfundComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [HubResourceCrowdfundComponent]
    }).compileComponents();

    fixture = TestBed.createComponent(HubResourceCrowdfundComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
